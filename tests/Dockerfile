FROM rust

# Create /src as root
RUN mkdir /src && chmod -R a+w /src

# Create non-root user
RUN useradd -ms /bin/bash nonroot
USER nonroot

# Create blank project
RUN USER=nonroot cargo new --bin /src/starship
WORKDIR /src/starship

# Copy the whole project
COPY . .

# "-Z unstable-options" is required for "--include-ignored"
CMD ["cargo", "test", "--", "-Z", "unstable-options", "--include-ignored"]
