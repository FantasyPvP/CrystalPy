from neuralintents import GenericAssistant
import socket
import selectors
import types
import sys
import log

sel = selectors.DefaultSelector()

class Crystal(GenericAssistant):
    def __init__(self, intents="./data/intents.json", port=9001, host="127.0.0.1", load=True, model_name="Crystal"):
        log.log("Starting Crystal", log.STATUS, colour=log.MAGENTA)
        super().__init__(intents, model_name=model_name)
        self.host = host
        self.port = port

        if load == True:
            try:
                self.load_model(model_name=model_name)
            except Exception as e:
            return

        self.train_model()
        self.save_model(model_name=model_name)


    def mainloop(self):

        log.log("Starting TCP Listener", log.STATUS)
        
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.bind((self.host, self.port))
            s.listen()
            s.setblocking(False)
            sel.register(s, selectors.EVENT_READ, data=None)
            try:
                while True:
                    events = sel.select(timeout=None)
                    for key, mask in events:
                        if key.data is None:
                            self.accept_wrapper(key.fileobj)
                        else:
                            self.service_connection(key, mask)
                            
            except KeyboardInterrupt:
                log.log("KEYBOARD INTERRUPT: EXITING", log.STATUS)
                
            except Exception as e:
                sel.close()
            

    def accept_wrapper(self, sock):
        conn, addr = sock.accept()
        conn.setblocking(False)
        data = types.SimpleNamespace(addr=addr, inb="", outb=b"")
        events = selectors.EVENT_READ | selectors.EVENT_WRITE
        sel.register(conn, events, data=data)

    def service_connection(self, key, mask):
        sock = key.fileobj
        data = key.data
        data.outb.decode("utf-8")

        if mask & selectors.EVENT_READ:
            recv_data = sock.recv(1024)
            if recv_data:
                response = self.process(recv_data.decode("utf-8"))
                data.outb += response.encode("utf-8")
            else:
                sel.unregister(sock)
                sock.close()
            
        if mask & selectors.EVENT_WRITE:
            if data.outb:
                sent = sock.send(data.outb)
                data.outb = data.outb[sent:]

    def process(self, data):
        print(data)
        if data.startswith("CHAT-REQUEST:"):
            log.log("Querying Crystal's AI", log.STATUS)
            return self.request(data[13:])



def main(): 
    kwargs = {}

    for idx, arg in enumerate(sys.argv):
        if "--host" in arg:
            kwargs["host"] = arg.split("=")[1]
        if "--port" in arg:
            kwargs["port"] = arg.split("=")[1]
        if "--intents" in arg:
            kwargs["intents"] = arg.split("=")[1]
        if "--retrain" in arg:
            kwargs["load"] = False
        if "--model" in arg:
            kwargs["model_name"] = arg.split("=")[1]

    crystal = Crystal(**kwargs)
    crystal.mainloop()
    

main()
