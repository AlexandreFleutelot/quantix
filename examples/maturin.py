import quantix

# Uses the lib.rs
# lib must be build and installed first with "maturin develop"

out = quantix.check_gpu()
print(out)