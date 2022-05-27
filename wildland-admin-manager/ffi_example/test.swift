print("Hello World Swift");

var custom_struct = get_custom_instance();
custom_struct.deref().print_foo();

for v in get_custom_instances_vector() {
    v.deref().print_foo()
}

print(return_string().toString());

for v in return_vec_string() {
    print(v.as_str().toString());
}

for v in return_vec_u8() {
    print(v);
}

print(return_u8());

var a = RustVec<RustString>();
a.push(value: RustString("Abc1"));
a.push(value: RustString("Abc2"));
var b = RustVec<UInt8>();
b.push(value: 66);
b.push(value: 77);
var c: UInt8 = 10;
var d = RustString("Asdf");
print_args(a, b, c, d);
