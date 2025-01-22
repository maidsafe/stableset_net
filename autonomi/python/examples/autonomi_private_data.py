from autonomi_client import Client, Wallet, PaymentOption
from typing import List, Optional
import json

class DataManager:
    def __init__(self, client: Client, wallet: Wallet):
        self.client = client
        self.wallet = wallet
        self.payment = PaymentOption.wallet(wallet)
        
    def store_private_data(self, data: bytes) -> str:
        """Store data privately and return its address"""
        addr = self.client.data_put(data, self.payment)
        return addr
        
    def retrieve_private_data(self, addr: str) -> bytes:
        """Retrieve privately stored data"""
        return self.client.data_get(addr)

def main():
    # Initialize
    private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    peers = ["/ip4/127.0.0.1/tcp/12000"]
    
    try:
        wallet = Wallet(private_key)
        client = Client.connect(peers)
        manager = DataManager(client, wallet)
        
        # Store private data
        user_data = {
            "username": "alice",
            "preferences": {
                "theme": "dark",
                "notifications": True
            }
        }
        private_data = json.dumps(user_data).encode()
        private_addr = manager.store_private_data(private_data)
        print(f"Stored private data at: {private_addr}")
        
        # Retrieve and verify private data
        retrieved_data = manager.retrieve_private_data(private_addr)
        retrieved_json = json.loads(retrieved_data.decode())
        print(f"Retrieved data: {retrieved_json}")
        
    except Exception as e:
        print(f"Error: {e}")
        return 1
        
    print("All operations completed successfully!")
    return 0

if __name__ == "__main__":
    exit(main()) 