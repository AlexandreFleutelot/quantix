use wgpu;
use pollster;

use pyo3::prelude::*;


#[pyfunction]
fn check_gpu() -> PyResult<()> {
    pollster::block_on(run_gpu_setup());
    Ok(())
}

#[pymodule]
fn quantix(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_gpu, m)?)?;
    Ok(())
}

pub async fn run_gpu_setup() {
    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        },
    ).await.expect("Echec de la recherche d'un adapter GPU");

    let info = adapter.get_info();
    println!("GPU selectionne : {:?} sur backend {:?}", info.name, info.backend);

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("quantixGPU"),
            required_features: wgpu::Features::TIMESTAMP_QUERY, // TODO: Pour mesurer les perf, a retirer plus tard
            required_limits: wgpu::Limits {
                max_storage_buffer_binding_size: 512 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        }).await.expect("Echec de la creation du device");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_setup() {
        pollster::block_on(run_gpu_setup());
    }
}