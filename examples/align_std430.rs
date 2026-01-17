use quantix::helpers;

use wgpu;

async fn test_std430() {
    let ctx = helpers::GpuContext::new().await;
}

#[tokio::main]
async fn main() {
    let _ = test_std430().await;
}