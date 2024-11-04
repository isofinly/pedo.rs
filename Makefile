# These are the patches to xv6 makefile
# Add them to your own makefile

RUSTC = rustc
RUST_TARGET = riscv64gc-unknown-none-elf
RUST_FLAGS = -C target-feature=+m,+a,+f,+d -C target-cpu=generic-rv64 --target=$(RUST_TARGET) \
             -C opt-level=2 -C debuginfo=2 -C link-arg=-nostartfiles -C link-arg=-static \
             -C linker=riscv64-unknown-elf-gcc \
             --edition=2021

UPROGS += $U/_goal

ULIB = $U/ulib.o $U/usys.o $U/printf.o $U/umalloc.o

$U/_goal: $U/rust/src/main.rs $U/user.ld $(ULIB)
	$(RUSTC) $(RUST_FLAGS) -o $U/rust/goal.o $< --emit=obj
	$(LD) -T $U/user.ld -o $@ $U/rust/goal.o $(ULIB)

UPROGS=\
	$U/_cat\
	$U/_echo\
	$U/_forktest\
	$U/_grep\
	$U/_init\
	$U/_kill\
	$U/_ln\
	$U/_ls\
	$U/_mkdir\
	$U/_rm\
	$U/_sh\
	$U/_stressfs\
	$U/_usertests\
	$U/_grind\
	$U/_wc\
	$U/_zombie\
	$U/_dumptests\
	$U/_dump2tests\
	$U/_alloctest\
	$U/_cowtest\
	$U/_lazytests\
	$U/_goal
