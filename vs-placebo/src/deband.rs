use const_str::cstr;
use foreign_types::ForeignType;
use libplacebo_rs::gpu::Tex;
use libplacebo_rs::shaders_root::ShaderObject;
use libplacebo_rs::{dispatch::Dispatch, log::Log, vulkan::Vulkan};
use libplacebo_sys::{
  pl_bit_encoding, pl_color_primaries, pl_color_repr, pl_color_space, pl_color_system,
  pl_color_transfer, pl_deband_params, pl_dispatch_params, pl_dither_method, pl_dither_params,
  pl_fmt_type, pl_frame, pl_hdr_metadata, pl_plane, pl_plane_data, pl_sample_src,
  pl_shader_obj_type, pl_shader_params, pl_tex_params, pl_tex_transfer_params, pl_vk_inst_params,
  pl_vulkan_params, PL_MAX_PLANES,
};
use miette::Result;
use std::ffi::CString;
use std::{
  ffi::{c_void, CStr},
  sync::Arc,
};
use vapoursynth4_rs::{
  core::CoreRef,
  ffi::VSSampleType,
  frame::{FrameContext, VideoFrame},
  key,
  map::{MapMut, MapRef},
  node::{
    ActivationReason, Dependencies, Filter as VsFilter, FilterDependency, Node, RequestPattern,
    VideoNode,
  },
  utils::bitblt,
};

#[allow(clippy::cast_sign_loss)]
fn get_planes_arg(planes: MapRef) -> Result<Vec<bool>, String> {
  let m = planes.num_elements(key!("planes")).unwrap_or(-1);
  let mut process = vec![m <= 0; 3];

  for i in 0..m {
    let o = planes
      .get_int_saturated(key!("planes"), i)
      .expect("Failed to read 'planes'.");

    if !(0..3).contains(&o) {
      return Err(format!("Plane index {o} is out of range [0, 3)."));
    }

    if process[o as usize] {
      return Err(format!("Plane {o} is specified more than once."));
    }

    process[o as usize] = true;
  }

  Ok(process)
}

pub struct Filter {
  node: VideoNode,

  /// Deband parameters.
  deband_params: pl_deband_params,

  /// Indicates whether or not the plane at index `i` should be processed.
  process_planes: Vec<bool>,

  dispatch: Dispatch,
  dither_state: ShaderObject,

  vulkan: Vulkan,
  pl_log: Arc<Log>,
}

impl Filter {
  fn create_textures(&self, data: &pl_plane_data, dst: &VideoFrame) -> (Tex, Tex) {
    let format = self
      .vulkan
      .plane_find_fmt(data)
      .expect("Failed to find a suitable texture format.");

    let tex_in = self.vulkan.tex_create(&pl_tex_params {
      w: data.width,
      h: data.height,
      format: format.as_ptr(),
      sampleable: true,
      host_writable: true,
      debug_tag: "tex_in".as_ptr().cast(),
      ..pl_tex_params::default()
    });

    let plane = data.component_map[0];
    let tex_out = self.vulkan.tex_create(&pl_tex_params {
      w: dst.frame_width(plane),
      h: dst.frame_height(plane),
      format: format.as_ptr(),
      renderable: true,
      host_readable: true,
      debug_tag: "tex_out".as_ptr().cast(),
      ..pl_tex_params::default()
    });

    (tex_in, tex_out)
  }

  #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  fn deband_frame(
    &self,
    frame_number: i32,
    src_img: &pl_frame,
    texes_in: &[Tex],
    texes_out: &[Tex],
  ) -> Result<()> {
    let mut dither_state =
      ShaderObject::new(self.vulkan.gpu(), pl_shader_obj_type::PL_SHADER_OBJ_DITHER);
    let dither_params = pl_dither_params {
      method: pl_dither_method::PL_DITHER_BLUE_NOISE,
      lut_size: 6,
      temporal: false,
      ..pl_dither_params::default()
    };

    for i in 0..src_img.num_planes as usize {
      let mut shader = self.dispatch.begin();
      shader.reset(&pl_shader_params {
        gpu: self.vulkan.gpu(),
        index: frame_number as u8,
        ..pl_shader_params::default()
      });

      let sample_src = pl_sample_src {
        tex: texes_in[i].as_ptr(),
        ..pl_sample_src::default()
      };

      shader.deband(&sample_src, &self.deband_params);

      // shader.dither(
      //   texes_out[i].format().component_depth[i],
      //   // &self.dither_state,
      //   &mut dither_state,
      //   &dither_params,
      // );

      let dispatch_result = self.dispatch.finish(&pl_dispatch_params {
        target: texes_out[i].as_ptr(),
        shader: &mut shader.as_ptr(),
        ..pl_dispatch_params::default()
      });

      match dispatch_result {
        Ok(()) => {}
        Err(error) => return Err(error),
      }
    }

    Ok(())
  }

  #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
  fn download_planes(
    &self,
    dst_image: &pl_frame,
    texes_out: &[Tex],
    vs_dst: &mut VideoFrame,
    plane_data: &[pl_plane_data],
  ) -> Result<()> {
    for i in 0..dst_image.num_planes as usize {
      let target_plane = &dst_image.planes[i];
      let vs_plane = target_plane.component_mapping[0];
      let out_format = texes_out[i].format();
      let dst_ptr = vs_dst.plane_mut(vs_plane);
      let dst_row_pitch: usize = (vs_dst.stride(vs_plane) / plane_data[i].pixel_stride as isize)
        as usize
        * out_format.texel_size;

      let download_result = self.vulkan.tex_download(&pl_tex_transfer_params {
        tex: texes_out[i].as_ptr(),
        row_pitch: dst_row_pitch,
        ptr: dst_ptr.cast(),
        ..pl_tex_transfer_params::default()
      });

      match download_result {
        Ok(()) => {}
        Err(error) => return Err(error),
      }
    }

    Ok(())
  }
}

impl VsFilter for Filter {
  type Error = CString;
  type FrameType = VideoFrame;
  type FilterData = ();

  #[allow(clippy::cast_possible_truncation)]
  fn create<'b>(
    input: MapRef<'_>,
    output: MapMut<'_>,
    _data: Option<Box<Self::FilterData>>,
    mut core: CoreRef,
  ) -> Result<(), Self::Error> {
    let Ok(node) = input.get_video_node(key!("clip"), 0) else {
      return Err(CString::new("Failed to get clip").unwrap());
    };

    let n = node.clone();
    let vi = n.info();

    if vi.format.bits_per_sample != 8
      && vi.format.bits_per_sample != 16
      && vi.format.bits_per_sample != 32
    {
      return Err(CString::new("placebo.Deband: input bit depth must be 8, 16, or 32.").unwrap());
    }

    let iterations = input.get_int(key!("iterations"), 0).unwrap_or(1) as i32;
    let threshold = input.get_float(key!("threshold"), 0).unwrap_or(3.0) as f32;
    let radius = input.get_float(key!("radius"), 0).unwrap_or(16.0) as f32;
    let grain = input.get_float(key!("grain"), 0).unwrap_or(4.0) as f32;

    let deband_params = pl_deband_params {
      iterations,
      threshold,
      radius,
      grain,
      ..pl_deband_params::default()
    };

    let process_planes = get_planes_arg(input).expect("Failed to determine places to process.");

    // libplacebo setup.

    // Log references are held by `Dispatch` and `Vulkan`.
    let pl_log = Arc::new(Log::default());

    let vulkan = Vulkan::new(
      &pl_log,
      &pl_vulkan_params {
        async_compute: true,
        async_transfer: true,
        queue_count: 1,
        instance_params: &pl_vk_inst_params {
          debug: true,
          ..pl_vk_inst_params::default()
        },
        ..pl_vulkan_params::default()
      },
    );

    let mut filter = Self {
      node,
      deband_params,
      process_planes,
      dispatch: Dispatch::new(&pl_log, &vulkan.gpu()),
      dither_state: ShaderObject::new(vulkan.gpu(), pl_shader_obj_type::PL_SHADER_OBJ_DITHER),
      // renderer: Renderer::new(&pl_log, &gpu),
      pl_log,
      vulkan,
    };

    let deps = [FilterDependency {
      source: filter.node.as_mut_ptr(),
      request_pattern: RequestPattern::StrictSpatial,
    }];

    core.create_video_filter(
      output,
      cstr!("Deband"),
      vi,
      Box::new(filter),
      Dependencies::new(&deps).unwrap(),
    );

    Ok(())
  }

  #[allow(clippy::cast_sign_loss)]
  fn get_frame(
    &self,
    n: i32,
    activation_reason: ActivationReason,
    _frame_data: *mut *mut c_void,
    mut ctx: FrameContext,
    core: CoreRef,
  ) -> Result<Option<VideoFrame>, Self::Error> {
    match activation_reason {
      ActivationReason::Initial => {
        ctx.request_frame_filter(n, &self.node);
      }
      ActivationReason::AllFramesReady => {
        let src = self.node.get_frame_filter(n, &mut ctx);

        let format = src.get_video_format();
        let height = src.frame_height(0);
        let width = src.frame_width(0);

        let mut dst = core.new_video_frame(format, width, height, Some(&src));

        let repr = pl_color_repr {
          bits: pl_bit_encoding {
            bit_shift: 0,
            color_depth: format.bits_per_sample,
            sample_depth: format.bits_per_sample,
          },
          sys: pl_color_system::PL_COLOR_SYSTEM_UNKNOWN,
          ..pl_color_repr::default()
        };

        let mut src_img = pl_frame {
          color: pl_color_space {
            primaries: pl_color_primaries::PL_COLOR_PRIM_UNKNOWN,
            transfer: pl_color_transfer::PL_COLOR_TRC_UNKNOWN,
            hdr: pl_hdr_metadata::default(),
          },
          repr,
          ..pl_frame::default()
        };
        let mut dst_img = src_img;

        let mut planes_data: Vec<pl_plane_data> = Vec::with_capacity(3);
        let mut proc_plane_idx: usize = 0;
        let mut texes_in: Vec<Tex> = Vec::with_capacity(PL_MAX_PLANES as usize);
        let mut texes_out: Vec<Tex> = Vec::with_capacity(PL_MAX_PLANES as usize);

        for plane in 0..format.num_planes {
          // Skip planes that weren't asked to be processed.
          if !self.process_planes[plane as usize] {
            unsafe {
              // Copy source plane to destination plane.
              bitblt(
                dst.plane_mut(plane).cast(),
                dst.stride(plane),
                src.plane(plane).cast(),
                src.stride(plane),
                (dst.frame_width(plane) * format.bytes_per_sample) as usize,
                dst.frame_height(plane) as _,
              );
            }
            continue;
          }

          // Add plane to the libplacebo frame.
          src_img.num_planes += 1;

          let data = pl_plane_data {
            type_: if format.sample_type == VSSampleType::Integer {
              pl_fmt_type::PL_FMT_UNORM
            } else {
              pl_fmt_type::PL_FMT_FLOAT
            },
            width: src.frame_width(plane),
            height: src.frame_height(plane),
            pixel_stride: format.bytes_per_sample as usize,
            row_stride: src.stride(plane) as usize,
            pixels: src.plane(plane).cast(),
            component_size: [format.bits_per_sample, 0, 0, 0],
            component_pad: [0; 4],
            component_map: [plane, 0, 0, 0],
            ..pl_plane_data::default()
          };

          planes_data.insert(proc_plane_idx, data);

          let (mut tex_in, tex_out) = self.create_textures(&data, &dst);

          src_img.planes[proc_plane_idx] = self
            .vulkan
            .upload_plane(&mut tex_in, &data)
            .expect("Failed to upload plane.");

          // HACK: `upload_plane()` may have changed the texture pointer.
          tex_in = unsafe { Tex::new_unchecked(src_img.planes[proc_plane_idx].texture.cast_mut()) };

          dst_img.planes[proc_plane_idx] = pl_plane {
            texture: tex_out.as_ptr(),
            components: tex_out.num_components(),
            component_mapping: [plane, 0, 0, 0],
            ..pl_plane::default()
          };

          texes_in.push(tex_in);
          texes_out.push(tex_out);

          proc_plane_idx += 1;
        }

        dst_img.num_planes = src_img.num_planes;

        let deband_result = self.deband_frame(n, &src_img, &texes_in, &texes_out);
        match deband_result {
          Ok(()) => {}
          Err(error) => return Err(CString::new(format!("{error:?}")).unwrap()),
        }

        let download_result = self.download_planes(&dst_img, &texes_out, &mut dst, &planes_data);
        match download_result {
          Ok(()) => {}
          Err(error) => return Err(CString::new(format!("{error:?}")).unwrap()),
        }

        for tex in texes_in.into_iter().chain(texes_out.into_iter()) {
          self.vulkan.tex_destroy(&tex);
        }

        return Ok(Some(dst));
      }
      ActivationReason::Error => {}
    }

    Ok(None)
  }

  const NAME: &'static CStr = cstr!("Deband");
  const ARGS: &'static CStr = cstr!(
    "clip:vnode;\
    planes:int[]:opt;"
  );
  const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;");
}
