# Hello World 

This repository demonstrates debugging with `sgx-gdb`, and memory measuring with `sgx_emmt`.

## Difference from [hello-world](./hello-world)
- all cargo build takes the debug mode
- gcc build with flag `-ggdb`, which will affects the the bridge/proxy libraris 
- add the `g_peak_heap_used` to version script [enclave.lds](enclave/enclave.lds)

## Memory measurement demo

```bash
cd ..
rm -rf build
mkdir build
cd build

cmake ..

make 

cd hello-world-debug/app/cargo/debug/

sgx-gdb ./app

enable sgx_emmt

r ../../../enclave/hello-world-debug-enclave.signed.so
```

Example output goes as 

```bash
Starting program: /workspace/build/hello-world-debug/app/cargo/debug/app ../../../enclave/hello-world-debug-enclave.signed.so
detect urts is loaded, initializing
[Thread debugging using libthread_db enabled]
Using host libthread_db library "/lib/x86_64-linux-gnu/libthread_db.so.1".
add-symbol-file '/workspace/build/hello-world-debug/enclave/hello-world-debug-enclave.signed.so' 0x7ffff55d8480 -readnow -s .interp 0x7ffff55d1270  -s .note.gnu.build-id 0x7ffff55d1280  -s .gnu.hash 0x7ffff55d12a8  -s .dynsym 0x7ffff55d12e0  -s .dynstr 0x7ffff55d1388  -s .gnu.version 0x7ffff55d13fa  -s .gnu.version_d 0x7ffff55d1408  -s .rela.dyn 0x7ffff55d1440  -s .plt 0x7ffff55d8460  -s .plt.got 0x7ffff55d8470  -s .nipx 0x7ffff56313a0  -s .rodata 0x7ffff5631fc0  -s .eh_frame_hdr 0x7ffff5638890  -s .eh_frame 0x7ffff563bda0  -s .gcc_except_table 0x7ffff56492dc  -s .tbss 0x7ffff584cb38  -s .init_array 0x7ffff584cb38  -s .fini_array 0x7ffff584cb40  -s .data.rel.ro 0x7ffff584cb60  -s .dynamic 0x7ffff584e520  -s .got 0x7ffff584e6d0  -s .got.plt 0x7ffff5850000  -s .data 0x7ffff5850020  -s .nipd 0x7ffff5850bd4  -s .niprod 0x7ffff5850c00  -s .bss 0x7ffff58514c0 
[+] Init Enclave Successful 163475045220354!
This is a normal world string passed into Enclave!
This is a in-Enclave Rust string!
[+] say_something success...
ecall_say_hello_to...
hello from ocall, sammyne
done ecall_say_hello_to
[+] ecall_say_hello_to success...
Enclave: "/workspace/build/hello-world-debug/enclave/hello-world-debug-enclave.signed.so"
  [Peak stack used]: 9 KB
  [Peak heap used]:  4 KB
remove-symbol-file -a 140737309934720
[Inferior 1 (process 38062) exited normally]
```

Just check the `[Peak stack used]` and `[Peak heap used]` line ~