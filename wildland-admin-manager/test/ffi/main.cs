using System;

namespace Main
{
    class Program
    {
        static void Main(string[] args)
        {
            Console.WriteLine("Hello World C#");
            
            var admin_manager = wildland.get_admin();
            admin_manager.print_foo();

            Console.WriteLine(wildland.return_string());
        
            var vec_string = wildland.return_vec_string();
            for(uint i=0; i< vec_string.size(); i++) {
                Console.WriteLine(vec_string.at(i).c_str());
            }

            var vec_u8 = wildland.return_vec_u8();
            for(uint i=0; i< vec_u8.size(); i++) {
                Console.WriteLine(vec_u8.at(i));
            }

            Console.WriteLine(wildland.return_u8());

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
}