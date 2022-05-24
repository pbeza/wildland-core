import wildland


print("Hello World Python")

# Python specific pointers deref function:
import ctypes
def deref_ptr(ptr, typ):
    return ctypes.cast(int(ptr), ctypes.POINTER(typ))[0]

#####

admin_manager = wildland.get_admin_instance()
admin_manager.deref().print_foo()

vector_admin_manager = wildland.get_admin_instances_vector()
vector_admin_manager.at(0).deref().print_foo()

#####

print(wildland.return_string().c_str())

#####

vec_string = wildland.return_vec_string()
for i in range(vec_string.size()):
    print(vec_string.at(i).c_str())

#####

vec_u8 = wildland.return_vec_u8()
for i in range(vec_u8.size()):
    print(deref_ptr(vec_u8.at(i), ctypes.c_ubyte))

#####

print(wildland.return_u8())

#####

a = wildland.StringVector()
a.push_back(wildland.RustString("Abc1"))
a.push_back(wildland.RustString("Abc2"))
b = wildland.ByteVector()
b.push_back(66)
b.push_back(77)
b.push_back(88)
c = 10
d = wildland.RustString("String from Python")
wildland.print_args(a, b, c, d)
