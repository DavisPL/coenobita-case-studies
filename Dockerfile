# Use the official Rust image as a base
FROM rust:latest

# Update package list and install Valgrind
RUN apt-get update && \
    apt-get install -y valgrind && \
    rm -rf /var/lib/apt/lists/*

# Set the default working directory for the container
WORKDIR /home

# By default, run an interactive shell; adjust as needed
CMD ["/bin/bash"]

