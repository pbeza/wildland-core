

public class main {
   public static void main(java.lang.String argv[]) {
     System.out.println("Hello World Java");
     System.loadLibrary("wildland");
     
     var custom_struct = wildland.get_custom_instance();
     custom_struct.deref().print_foo();

     var custom_struct_vector = wildland.get_custom_instances_vector();
     custom_struct_vector.at(0).deref().print_foo();
     
     System.out.println(wildland.return_string().c_str());
     
     var vec_string = wildland.return_vec_string();
     for(long i=0; i< vec_string.size(); i++) {
       System.out.println(vec_string.at(i).c_str());
     }

     var vec_u8 = wildland.return_vec_u8();
     for(long i=0; i< vec_u8.size(); i++) {
       System.out.println(vec_u8.at(i));
     }

      System.out.println(wildland.return_u8());

      var a = new StringVector();
      a.push_back(new RustString("Abc1"));
      a.push_back(new RustString("Abc2"));
      var b = new ByteVector();
      b.push_back((byte)66);
      b.push_back((byte)77);
      byte c = 10;
      var d = new RustString("Asdf");
      wildland.print_args(a, b, c, d);

   }
 }
