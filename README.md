# rginx
Fast Webserver in Rust

The goal of the project is to develop a high performance web server in an upcoming systems language - Rust. The web server architecture grew from a multi-threaded pool based implementation to a fully event driven architecture. The event driven architecture is an implementation of the Reactor pattern. The web server aims to be as performant as the more popular web servers like Apache Httpd and NGINX. By developing the webserver in a language which was built from the start keeping in mind the memory safety issues of C/C++ while maintaining performance, it helped us explore paradigms which are interesting and really effective in writing safe code.

Current implementation attempts a fast FileServer. The performance comparisons can be found in the [report](https://github.com/deyb/rginx/blob/master/report/report.pdf)

Pre-requisites:
1. Rust (v1.22+)
2. Cargo (0.23+)

To run the program:
    
    cargo run 8080 /tmp/ 3

starts the webserver at port:8080, serves the /tmp directory with 3 threads apart from the main thread
    
