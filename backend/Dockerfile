FROM golang:1.26-alpine AS build
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /app ./cmd/app

FROM alpine:3.20
WORKDIR /data
COPY --from=build /app /usr/local/bin/app
EXPOSE 8080
ENTRYPOINT ["app"]
