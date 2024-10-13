use std::sync::Arc;
use wgpu::util::DeviceExt;

static WORK_GROUP_MAX_X: f32 = 256.0;

#[repr(C)]
#[derive(Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct OutputVertex {
  position: [f32; 2],
  normal: [f32; 2],
}

pub struct LineTessellation {
  /// wgpu device and queue pair
  device_queue: (Arc<wgpu::Device>, Arc<wgpu::Queue>),

  /// wgpu pipeline
  pipeline: wgpu::ComputePipeline,

  /// bind group layout
  bind_group_layout: wgpu::BindGroupLayout,
}

impl LineTessellation {
  pub fn new((device, queue): (Arc<wgpu::Device>, Arc<wgpu::Queue>)) -> Self {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
        "shader/line.wgsl"
      ))),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 3,
          visibility: wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      entry_point: Some("main"),
      module: &shader_module,
      compilation_options: wgpu::PipelineCompilationOptions::default(),
      cache: None,
    });

    Self {
      device_queue: (device, queue),
      pipeline,
      bind_group_layout,
    }
  }

  fn create_buffers(&self, vertices: &[f32], indices: &[u32]) -> [wgpu::Buffer; 4] {
    let (device, _) = &self.device_queue;

    // generate 4 vertices for each edge and 6 indices for each edge
    let line_vertices_buffer_size =
      (std::mem::size_of::<OutputVertex>() * (indices.len()) * 4) as u64;
    let line_indices_buffer_size = (std::mem::size_of_val(indices) * 6) as u64;

    [
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::STORAGE, // for the compute shader
      }),
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::STORAGE, // for the compute shader
      }),
      device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        usage: wgpu::BufferUsages::VERTEX // reuse as vertex buffer later
          | wgpu::BufferUsages::STORAGE // for the compute shader
          | wgpu::BufferUsages::COPY_SRC, // for debug and test purposes
        size: line_vertices_buffer_size,
        mapped_at_creation: false,
      }),
      device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        usage: wgpu::BufferUsages::INDEX // reuse as index buffer later
          | wgpu::BufferUsages::STORAGE // for the compute shader
          | wgpu::BufferUsages::COPY_SRC, // for debug and test purposes
        size: line_indices_buffer_size,
        mapped_at_creation: false,
      }),
    ]
  }

  fn create_bind_group(
    &self,
    vertices_buffer: &wgpu::Buffer,
    indices_buffer: &wgpu::Buffer,
    line_vertices_buffer: &wgpu::Buffer,
    line_indices_buffer: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    let (device, _) = &self.device_queue;

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &self.bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: vertices_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: indices_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: line_vertices_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 3,
          resource: line_indices_buffer.as_entire_binding(),
        },
      ],
    })
  }

  // TODO: maybe later a variant with VERTEX and INDEX buffer as input params
  pub async fn tessellate(
    &self,
    (vertices, indices): (&[f32], &[u32]),
  ) -> (wgpu::Buffer, wgpu::Buffer) {
    let (device, queue) = &self.device_queue;

    #[rustfmt::skip]
    let [
      vertices_buffer,
      indices_buffer,
      line_vertices_buffer,
      line_indices_buffer
    ] = self.create_buffers(vertices, indices);

    let bind_group = self.create_bind_group(
      &vertices_buffer,
      &indices_buffer,
      &line_vertices_buffer,
      &line_indices_buffer,
    );

    let mut command_encoder =
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
      let mut pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

      pass.set_pipeline(&self.pipeline);
      pass.set_bind_group(0, Some(&bind_group), &[]);

      let x = (indices.len() as f32 / WORK_GROUP_MAX_X).ceil() as u32;
      pass.dispatch_workgroups(x, 1, 1);
    } // out of scope

    queue.submit(Some(command_encoder.finish()));

    (line_vertices_buffer, line_indices_buffer)
  }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
  use super::*;
  use log::info;

  #[repr(C)]
  #[derive(Debug, Default, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
  struct Vertex {
    position: [f32; 2],
    normal: [f32; 2],
  }

  async fn initialize_test() -> (wgpu::Device, wgpu::Queue) {
    env_logger::init_from_env(
      env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all()),
      ..Default::default()
    });

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
      })
      .await
      .unwrap();

    adapter
      .request_device(&wgpu::DeviceDescriptor::default(), None)
      .await
      .unwrap()
  }

  async fn map_and_log_buffer<F>(
    (device, queue): (Arc<wgpu::Device>, Arc<wgpu::Queue>),
    src_buffer: &wgpu::Buffer,
    src_buffer_size: u64,
    from_bytes: F,
  ) where
    F: Fn(&[u8]),
  {
    let dest_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      size: src_buffer_size,
      mapped_at_creation: false,
    });

    let mut command_encoder =
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    command_encoder.copy_buffer_to_buffer(src_buffer, 0, &dest_buffer, 0, src_buffer_size);

    let submission_index = queue.submit(Some(command_encoder.finish()));

    let (sender, receiver) = futures::channel::oneshot::channel();
    let buffer_slice = dest_buffer.slice(..);

    buffer_slice.map_async(wgpu::MapMode::Read, move |v| match sender.send(v) {
      Ok(_) => (),
      Err(error) => match error {
        Ok(_) => (),
        Err(error) => {
          panic!("{error:?}")
        }
      },
    });

    device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index)); // has no effect for web target

    match receiver.await {
      Ok(rec_result) => match rec_result {
        Ok(_) => {
          {
            let bytes = buffer_slice.get_mapped_range();
            from_bytes(&bytes[..]);
          } // out of scope
          dest_buffer.unmap();
        }
        Err(error) => panic!("{error:?}"),
      },
      Err(error) => panic!("{error:?}"),
    }
  }

  #[test]
  fn compute_lines() {
    let (device, queue) = pollster::block_on(initialize_test());
    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let vertices = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
    let indices = [0, 1, 2, 3, 0];

    let line_tessellation = Arc::new(LineTessellation::new((device.clone(), queue.clone())));

    let line_tessellation1 = line_tessellation.clone();
    let device1 = device.clone();
    let queue1 = queue.clone();
    let handle1 = std::thread::spawn(move || {
      let (vertices, indices) =
        pollster::block_on(line_tessellation1.tessellate((&vertices, &indices)));
      {
        pollster::block_on(map_and_log_buffer(
          (device1.clone(), queue1.clone()),
          &vertices,
          vertices.size(),
          |bytes| {
            let size = std::mem::size_of::<Vertex>();
            for i in 0..16 {
              let start = i * size;
              let end = start + size;
              let output: Vertex = *bytemuck::from_bytes(&bytes[start..end]);
              info!("{:?}", output);
            }
          },
        ));
        pollster::block_on(map_and_log_buffer(
          (device1, queue1),
          &indices,
          indices.size(),
          |bytes| {
            let size = std::mem::size_of::<u32>();
            let indices: Vec<u32> = (0..24)
              .enumerate()
              .map(|(i, _)| {
                let start = i * size;
                let end = start + size;
                *bytemuck::from_bytes(&bytes[start..end])
              })
              .collect();
            info!("{:?}", indices);
          },
        ));
      }
    });
    let handle2 = std::thread::spawn(move || {
      let (vertices, indices) =
        pollster::block_on(line_tessellation.tessellate((&vertices, &indices)));
      {
        pollster::block_on(map_and_log_buffer(
          (device.clone(), queue.clone()),
          &vertices,
          vertices.size(),
          |_| {},
        ));
        pollster::block_on(map_and_log_buffer(
          (device, queue),
          &indices,
          indices.size(),
          |_| {},
        ));
      }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
  }
}
