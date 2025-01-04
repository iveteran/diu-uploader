# forward http request/response with upload_server.py, this tool will print all request and response, it's useful for watching all traffic
echo "Listening on 8099 and forward to localhost:7777"
socat -v -v tcp-listen:8099,reuseaddr,fork tcp:localhost:7777
