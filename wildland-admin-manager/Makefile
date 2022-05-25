.PHONY: prepare java csharp clean


# Override the following paths in order
# to point the proper folder with inlcude files:
JDK_INC_DIR=/usr/lib/jvm/java-17-openjdk-17.0.3.0.7-1.fc35.x86_64/include
PYTHON_DIR=/usr/include/python3.10/



# Those paths should not be changed:
CXX_OUT=../target/cxxbridge/wildland-admin-manager/src/ffi
CXX_RUST=../target/cxxbridge/rust


prepare:
	cargo build --features "cxx_binding"
	mkdir -p cpp
	cp $(CXX_OUT)/mod.rs.h cpp/mod.rs.h
	cp $(CXX_OUT)/mod.rs.cc cpp/mod.rs.cc
	cp $(CXX_RUST)/cxx.h cpp/cxx.h
	##############################
	##### SWIG Workarounds: ######
	##############################
	# * Swig doesn't like `final` keyword:
		sed -i 's/final//g' cpp/mod.rs.h
	# * Inline namespaces also make problems:
		sed -i 's/::rust/::rust::cxxbridge1/g' cpp/mod.rs.h
	# * Every param between `[[`  `]]` is unparsable:
		sed -i 's/\[\[noreturn\]\]//g' cpp/mod.rs.h
	# * Not all variadic `...` are supported:
		sed -i 's/\.\.\.//g' cpp/mod.rs.h

java: prepare
	mkdir -p wildland_java
	cd cpp ; swig -java -c++ -w516,503,476,302 -outdir ../wildland_java ../wildland.i
	mv wildland_wrap.cxx cpp/wildland_wrap.cxx
	cd cpp ; g++ -fpermissive -shared -fPIC --std=c++14 -w \
		wildland_wrap.cxx mod.rs.cc \
		-L../../target/debug \
		-lwildland_admin_manager \
		-I$(JDK_INC_DIR) \
		-I$(JDK_INC_DIR)/linux \
		-o ../wildland_java/libwildland.so

java_test: java
	cp test/ffi/test.java wildland_java/test.java
	cd wildland_java ; javac test.java
	cd wildland_java ; java -Djava.library.path="." main

csharp: prepare
	mkdir -p wildland_csharp
	cd cpp ; swig -csharp -c++ -w516,503,476,302 -outdir ../wildland_csharp ../wildland.i
	mv wildland_wrap.cxx cpp/wildland_wrap.cxx
	cd cpp ; g++ -fpermissive -shared -fPIC --std=c++14 -w \
		wildland_wrap.cxx mod.rs.cc \
		-L../../target/debug \
		-lwildland_admin_manager \
		-o ../wildland_csharp/libwildland.so

csharp_test: csharp
	cp test/ffi/test.cs wildland_csharp/test.cs
	cd wildland_csharp ; mcs -out:test.exe *.cs
	cd wildland_csharp ; mono test.exe

python: prepare
	mkdir -p wildland_python
	cd cpp ; swig -python -c++ -w516,503,476,302,383,509 -outdir ../wildland_python ../wildland.i
	mv wildland_wrap.cxx cpp/wildland_wrap.cxx
	cd cpp ; g++ -fpermissive -shared -fPIC --std=c++14 -w \
		wildland_wrap.cxx mod.rs.cc \
		-L../../target/debug \
		-lwildland_admin_manager \
		-I$(PYTHON_DIR) \
		-o ../wildland_python/_wildland.so

python_test: python
	cp test/ffi/test.py wildland_python/test.py
	cd wildland_python ; python test.py

swift:
	mkdir -p wildland_swift
	SWIFT_BRIDGE_OUT_DIR="$(CURDIR)/wildland_swift" cargo build --features "swift_binding"
	cp test/ffi/test.swift wildland_swift/main.swift
	swiftc -L ../target/debug -lwildland_admin_manager \
		-I wildland_swift -import-objc-header \
		swift_ffi/bridging-header.h \
		./wildland_swift/SwiftBridgeCore.swift \
		./wildland_swift/wildland/wildland.swift \
		wildland_swift/main.swift \
		-o wildland_swift/swift_app

swift_test: swift
	@echo ""
	@echo "Result from the Swift test application:"
	@echo ""
	./wildland_swift/swift_app


clean:
	rm -r wildland_java wildland_csharp wildland_python cpp wildland_swift
