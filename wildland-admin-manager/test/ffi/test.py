import wildland


print("TODO: Python FFI Testsuite")

# Python specific pointers deref function:
import ctypes
def deref_ptr(ptr, typ):
    return ctypes.cast(int(ptr), ctypes.POINTER(typ))[0]
