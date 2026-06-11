import torch
import warnings

# Suppress ONNX trace warnings 
warnings.filterwarnings("ignore")

print("Downloading MegaLoc model...")
model = torch.hub.load("gmberton/MegaLoc", "get_trained_model")
model.eval()

# Create a dummy tensor (Batch, Channels, Height, Width) 
# We use 322x322 as a baseline, but the dynamic_axes config allows arbitrary sizes later
dummy_input = torch.randn(1, 3, 322, 322)

onnx_file_path = "megaloc.onnx"
print(f"Compiling to {onnx_file_path}...")

# Export the computational graph
torch.onnx.export(
    model,
    dummy_input,
    onnx_file_path,
    export_params=True,
    opset_version=14,  # Crucial: Opset 14+ is required for the DINOv2 Transformer layers
    do_constant_folding=True,
    input_names=['input_image'],
    output_names=['descriptor'],
    dynamic_axes={
        'input_image': {0: 'batch_size', 2: 'height', 3: 'width'},
        'descriptor': {0: 'batch_size'}
    }
)

print("Export complete. You can now delete Python from your production stack.")