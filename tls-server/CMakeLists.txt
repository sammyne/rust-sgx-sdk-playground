cmake_minimum_required(VERSION 3.10)

project(tls-server)

#set(appPath ${CMAKE_CURRENT_SOURCE_DIR}/app)
set(enclavePath ${CMAKE_CURRENT_SOURCE_DIR}/enclave)

#set(enclave enclave)
set(app ${PROJECT_NAME}-app)
set(enclave ${PROJECT_NAME}-enclave)

set(config ${enclavePath}/enclave.config.xml)
set(edl enclave.edl)
set(enclaveLib ${PROJECT_NAME}-enclave.signed.so)
set(key ${enclavePath}/isv.pem)

add_subdirectory(enclave)
add_subdirectory(app)
