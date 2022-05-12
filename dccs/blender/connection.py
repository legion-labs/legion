from sonora.protocol import WebRpcError
import sonora.client
from preferences import get_preferences

class Connection:
    def __init__(self, context):
        self.context = context

    def __enter__(self):
        preferences = get_preferences(self.context)
        server_address = preferences.server_address
        self.channel = sonora.client.insecure_web_channel(server_address)
        self.timeout = preferences.timeout
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        print("exc_type {}".format(exc_type))
        print("exc_value {}".format(exc_value))
        print("traceback {}".format(traceback))
