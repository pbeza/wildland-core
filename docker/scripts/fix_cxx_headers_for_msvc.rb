# This file is an attempt to make the MSVC build pass
#
# It is used as a replacement for POSIX `sed` which is a nightmare to use in PowerShell. 

f = File.read('ffi_cxx.h')

f.gsub!(/typedef long unsigned int uintptr_t;/, "")
f.gsub!(/RustStr s = RustStr/, "RustStr s")
f.gsub!(/this->str.c_str\(\),/, "reinterpret_cast<uint8_t*>(\&this->str[0]),")
f.gsub!(/(bool\(.+),\n.+?true/, "\\1")
f.gsub!(/(void\(.+),\n.+?true/, "\\1")
f.gsub!(/(u32\(.+),\n.+?true/, "\\1")

File.write('ffi_cxx.h', f)
