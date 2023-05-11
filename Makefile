# Makefile to build releases and check build reproducibility

.PHONY: repro release uf2 repro_check
PLATFORM:=ledmatrix
RELEASE_BIN:=target/thumbv6m-none-eabi/release/$(PLATFORM)
TARGET_FOLDER:=target/thumbv6m-none-eabi/release
FIRST_BIN:=$(PLATFORM)_first

clean:
	rm -f $(RELEASE_BIN)

# Simple reproducibility check
# Run clean build twice
repro:
	# First build
	cargo clean
	$(MAKE) release

	# Back up, so that `cargo clean` won't remove it
	cp $(RELEASE_BIN) $(FIRST_BIN)
	# Sleep a bit to make sure timestamps are different
	sleep 2

	# Second build
	cargo clean
	$(MAKE) release

	# Move back into target, so that `cargo clean` can remove it next time
	mv $(FIRST_BIN) $(TARGET_FOLDER)

	$(MAKE) repro_check

repro_check:
	# Make sure the username wasn't embedded
	! strings $(RELEASE_BIN) | grep $(shell whoami)

	# Check that both binaries are equivalent
	sha256sum $(RELEASE_BIN) $(TARGET_FOLDER)/$(FIRST_BIN)
	cmp $(RELEASE_BIN) $(TARGET_FOLDER)/$(FIRST_BIN)

release: $(RELEASE_BIN)

uf2: $(RELEASE_BIN)
	elf2uf2-rs target/thumbv6m-none-eabi/release/$(PLATFORM) $(PLATFORM).uf2
	ls -lh $(PLATFORM).uf2
	sha256sum $(PLATFORM).uf2


# Build release binary
# Need to remap paths to avoid local absolute paths being embedded in the binary
$(RELEASE_BIN):
	# Need to provide the rustflags defined in .cargo/config.toml again because
	# setting the environment variable overrides them
	env \
		RUSTFLAGS="--remap-path-prefix=$$PWD=. --remap-path-prefix=$$CARGO_HOME=home --remap-path-prefix=$$HOME=home -C link-arg=--nmagic -C link-arg=-Tlink.x -C link-arg=-Tdefmt.x -C linker=flip-link -C inline-threshold=5 -C no-vectorize-loops" \
		cargo build --release -p $(PLATFORM)

	ls -lh $(RELEASE_BIN)
	sha256sum $(RELEASE_BIN)
