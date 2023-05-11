# Makefile to build releases and check build reproducibility

.PHONY: repro
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

# Build release binary
# Need to remap paths to avoid local absolute paths being embedded in the binary
$(RELEASE_BIN):
	cargo build --release -p $(PLATFORM)

	# TODO: Doesn't work, produces a 416B binary, instead of the 46KB binary without this
	#env \
	#	RUSTFLAGS="--remap-path-prefix=$$PWD=. --remap-path-prefix=$$CARGO_HOME=home --remap-path-prefix=$$HOME=home" \
	#	cargo build --release -p $(PLATFORM)
	# Manually remap
	./remap-path-prefix.py $(RELEASE_BIN)

	ls -lh $(RELEASE_BIN)
	sha256sum $(RELEASE_BIN)
