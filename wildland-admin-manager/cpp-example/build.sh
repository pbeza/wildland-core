g++ -std=c++20 -o out main.cpp \
../../target/cxxbridge/wildland-admin-manager/src/ffi/mod.rs.cc \
-I ../../target/cxxbridge/ \
-L ../../target/debug -lwildland_admin_manager
