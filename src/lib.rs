pub mod helpers;

use pyo3::prelude::*;
use pollster;

#[pyfunction]
fn check_gpu() -> PyResult<String> {
    let ctx = pollster::block_on(helpers::GpuContext::new());
    Ok(format!(
        "Quantix lié au GPU : {:?} ({:?})",
        ctx.adapter_info.name, ctx.adapter_info.backend
    ))
}

#[pymodule]
fn quantix(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_gpu, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_setup() {
       let ret = check_gpu();
        println!("{}",ret.unwrap())
    }
}