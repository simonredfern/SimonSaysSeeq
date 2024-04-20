# me - this DAT
#
# dat - the DAT that received the data
# rowIndex - is the row number the data was placed into
# message - an ascii representation of the data
#           Unprintable characters and unicode characters will
#           not be preserved. Use the 'bytes' parameter to get
#           the raw bytes that were sent.
# bytes - a byte array of the data received
# peer - a Peer object describing the originating message
#   peer.close()    #close the connection
#   peer.owner  #the operator to whom the peer belongs
#   peer.address    #network address associated with the peer
#   peer.port       #network port associated with the peer
#

import struct

def onReceive(dat, rowIndex, message, bytes, peer):
	# grab DAT
	t1 = op('texttop6')
    # we know we are being sent a 4 byte unsigned integer (C++ uint64_t) 
	# doesn't work int_values = struct.unpack('<Q', bytes) #little endian. Thanks chatgpt
	try:
		# doesn't work int_values = struct.unpack('<Q', bytes) #little endian. Thanks chatgpt
		# works except for 8000000 ?
		int_values = struct.unpack('<i', bytes)
		# doesn't work int_values = struct.unpack('!Q', bytes)
		first_value = int_values[0]
		print(first_value)
		t1.par.text = first_value
	except:
		print("could not decode:")
		print(bytes)
	return