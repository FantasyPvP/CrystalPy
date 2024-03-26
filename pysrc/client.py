import socket

def send_request(message):
	with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
		try:
			s.connect(("127.0.0.1", 9000))
		except Exception as e:
			print("connection failed:\n", e)
			return
		s.sendall(message.encode("utf-8"))
		data = s.recv(1024)

	print(data.decode("utf-8"))


while True:
	print(send_request(input(" > ")))
