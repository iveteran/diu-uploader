#!/bin/env python3
import http.server
import socketserver
import io
import cgi
from urllib.parse import parse_qs

# Change this to serve on a different port
#PORT = 8099
PORT = 7777

class CustomHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):

    def do_POST(self):        
        r, info = self.deal_post_data()
        print(r, info, "by: ", self.client_address)
        f = io.BytesIO()
        if r:
            f.write(b"Success\n")
            self.send_response(200)
        else:
            f.write(b"Failed\n")
            self.send_response(500)
        length = f.tell()
        f.seek(0)
        self.send_header("Content-type", "text/plain")
        self.send_header("Content-Length", str(length))
        self.end_headers()
        if f:
            self.copyfile(f, self.wfile)
            f.close()      

    def deal_post_data(self):
        ctype, pdict = cgi.parse_header(self.headers['Content-Type'])
        if 'boundary' in pdict:
            pdict['boundary'] = bytes(pdict['boundary'], "utf-8")
        content_length = int(self.headers['Content-Length'])
        pdict['CONTENT-LENGTH'] = content_length
        print("ctype: ", ctype)
        if ctype == 'multipart/form-data':
            form = cgi.FieldStorage(
                    fp=self.rfile,
                    headers=self.headers,
                    environ={'REQUEST_METHOD':'POST', 'CONTENT_TYPE':self.headers['Content-Type'], })
            print("form: ", form)
            for part_name in form:
                print("-" * 50)
                print("form item name:", part_name)
                file_form = form[part_name]
                print("form item:", file_form)
                try:
                    if isinstance(file_form, list):
                        for record in file_form:
                            if record.filename:
                                open("./%s" % record.filename, "wb").write(record.file.read())
                            else:
                                print("form item content: ", record.file.read())
                    else:
                        if file_form.filename:
                            open("./%s" % file_form.filename, "wb").write(file_form.file.read())
                        else:
                            print("form item content: ", file_form.file.read())
                except IOError:
                    return (False, "Can't create file to write, do you have permission to write?")
        elif ctype == 'application/x-www-form-urlencoded':
            postvars = parse_qs(self.rfile.read(content_length))
            print("post values: ", postvars)
        elif ctype == "text/plain":
            print("body content: ", self.rfile.read(content_length))
        else:
            return (False, "Unsupported content-type: {}".format(ctype))

        print("-" * 50)

        return (True, "Files uploaded")

Handler = CustomHTTPRequestHandler
with socketserver.TCPServer(("", PORT), Handler) as httpd:
    print("serving at port", PORT)
    httpd.serve_forever()

#
# Usage: 
#  server side: ./upload_server.py
#  client side: curl -F 'files=@<FILENAME>' http://localhost:8099/
#
