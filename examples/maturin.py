import quantix

# Uses the lib.rs
# lib must be build and installed first with "maturin develop"
def main():
    out = quantix.check_gpu()
    print(out)

if __name__ == "__main__":
    main()