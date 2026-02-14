from gradio_client import Client
import json

try:
    client = Client("zomehwh/GPT-SoVITS-ba")
    print("Endpoints found:")
    # print(client.view_api(all_endpoints=True))
    # We want to see the predict endpoint specifically
    # Gradio client prints this to stdout when initialized or viewed
except Exception as e:
    print(f"Error: {e}")
