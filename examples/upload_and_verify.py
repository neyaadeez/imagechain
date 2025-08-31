#!/usr/bin/env python3
"""
Example script to demonstrate uploading a file and verifying its manifest.
"""
import argparse
import json
import os
import requests

def upload_file(server_url, file_path):
    """Upload a file to the server and return the manifest."""
    url = f"{server_url}/api/upload"
    
    with open(file_path, 'rb') as f:
        files = {'file': (os.path.basename(file_path), f)}
        response = requests.post(url, files=files)
    
    response.raise_for_status()
    return response.json()

def verify_manifest(server_url, manifest):
    """Verify a manifest against the server."""
    url = f"{server_url}/api/verify"
    headers = {'Content-Type': 'application/json'}
    
    response = requests.post(url, headers=headers, json=manifest)
    response.raise_for_status()
    return response.json()

def main():
    parser = argparse.ArgumentParser(description='Upload and verify files with ImageChain')
    parser.add_argument('file_path', help='Path to the file to upload')
    parser.add_argument('--server', default='http://localhost:3000', 
                       help='Server URL (default: http://localhost:3000)')
    
    args = parser.parse_args()
    
    try:
        # Step 1: Upload the file and get the manifest
        print(f"Uploading {args.file_path}...")
        result = upload_file(args.server, args.file_path)
        
        if not result.get('success'):
            print(f"Error: {result.get('error', 'Unknown error')}")
            return
            
        manifest = result.get('data', {})
        print("\nManifest received:")
        print(json.dumps(manifest, indent=2))
        
        # Save the manifest to a file
        manifest_path = f"{os.path.splitext(args.file_path)[0]}_manifest.json"
        with open(manifest_path, 'w') as f:
            json.dump(manifest, f, indent=2)
        print(f"\nManifest saved to {manifest_path}")
        
        # Step 2: Verify the manifest
        print("\nVerifying manifest...")
        verify_result = verify_manifest(args.server, manifest)
        print("Verification result:", verify_result)
        
    except requests.exceptions.RequestException as e:
        print(f"Error: {e}")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")

if __name__ == "__main__":
    main()
