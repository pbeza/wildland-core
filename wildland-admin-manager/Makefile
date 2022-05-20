.PHONY: prepare java csharp clean


# Override the following paths in order
# to point the proper folder with inlcude files:
JDK_INC_DIR=/usr/lib/jvm/java-17-openjdk-17.0.3.0.7-1.fc35.x86_64/include
PYTHON_DIR=/usr/include/python3.10/



# Those paths should not be changed:
CXX_OUT=../target/cxxbridge/wildland-admin-manager/src
CXX_RUST=../target/cxxbridge/rust


prepare:
	cargo build
	mkdir -p cpp
	cp $(CXX_OUT)/lib.rs.h cpp/lib.rs.h
	cp $(CXX_OUT)/lib.rs.cc cpp/lib.rs.cc
	cp $(CXX_RUST)/cxx.h cpp/cxx.h
	cp cxx.cc cpp/cxx.cc

	##### Workarounds: ######

	# Swig doesn't like `final` keyword:
	sed -i 's/final//g' cpp/lib.rs.h

	# Inline namespaces also make problems:
	sed -i 's/::rust/::rust::cxxbridge1/g' cpp/lib.rs.h

	# Every param between `[[`  `]]` is unparsable
	sed -i 's/\[\[noreturn\]\]//g' cpp/lib.rs.h

java: prepare
	mkdir -p wildland_java
	cd cpp ; swig -java -c++ -outdir ../wildland_java ../wildland.i
	mv wildland_wrap.cxx cpp/wildland_wrap.cxx
	cd cpp ; g++ -fpermissive -shared -fPIC --std=c++14 \
		wildland_wrap.cxx cxx.cc lib.rs.cc \
		-L../../target/debug \
		-lwildland_admin_manager \
		-I$(JDK_INC_DIR) \
		-I$(JDK_INC_DIR)/linux \
		-o ../wildland_java/libwildland.so

java_test: java
	cp test/ffi/main.java wildland_java/main.java
	cd wildland_java ; javac main.java
	cd wildland_java ; java -Djava.library.path="/home/tkulik/repos/wildland-core/wildland-admin-manager/wildland_java" main

csharp: prepare
	mkdir -p wildland_csharp
	cd cpp ; swig -csharp -c++ -outdir ../wildland_csharp ../wildland.i
	mv wildland_wrap.cxx cpp/wildland_wrap.cxx
	cd cpp ; g++ -fpermissive -shared -fPIC --std=c++14 \
		wildland_wrap.cxx cxx.cc lib.rs.cc \
		-L../../target/debug \
		-lwildland_admin_manager \
		-o ../wildland_csharp/libwildland.so

csharp_test: csharp
	cp test/ffi/main.cs wildland_csharp/main.cs
	cd wildland_csharp ; mcs -out:main.exe *.cs
	cd wildland_csharp ; mono main.exe

python: prepare
	mkdir -p wildland_python
	cd cpp ; swig -python -c++ -outdir ../wildland_python ../wildland.i
	mv wildland_wrap.cxx cpp/wildland_wrap.cxx
	cd cpp ; g++ -fpermissive -shared -fPIC --std=c++14 \
		wildland_wrap.cxx cxx.cc lib.rs.cc \
		-L../../target/debug \
		-lwildland_admin_manager \
		-I$(PYTHON_DIR) \
		-o ../wildland_python/_wildland.so

python_test: python
	cp test/ffi/main.py wildland_python/main.py
	cd wildland_python ; python main.py

clean:
	rm -r wildland_java wildland_csharp wildland_python
