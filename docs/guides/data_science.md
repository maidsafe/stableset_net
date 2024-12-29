# Data Science with Autonomi

This guide covers how to use Autonomi for data science applications, including data storage, processing, and analysis.

## Overview

Autonomi provides a secure and decentralized platform for data science applications. This guide demonstrates how to leverage Autonomi's features for data storage, processing, and analysis while maintaining data security and privacy.

## Getting Started

### Prerequisites

- Python 3.8+
- Autonomi Python client
- Common data science libraries (numpy, pandas, scikit-learn)

### Installation

```bash
pip install autonomi-client
pip install numpy pandas scikit-learn
```

## Data Storage

### Storing Datasets

```python
from autonomi import Client
import pandas as pd
from typing import Optional

async def store_dataset(
    df: pd.DataFrame,
    name: str,
    description: Optional[str] = None
) -> str:
    client = await Client.init()
    
    # Convert DataFrame to bytes
    data = df.to_parquet()
    
    # Store metadata
    metadata = {
        'name': name,
        'description': description,
        'columns': list(df.columns),
        'shape': df.shape,
        'dtypes': {str(k): str(v) for k, v in df.dtypes.items()}
    }
    
    # Store data and metadata
    data_pointer = await client.store_chunk(data)
    metadata_pointer = await client.store_scratch_pad(metadata)
    
    # Link data and metadata
    dataset = {
        'data': data_pointer,
        'metadata': metadata_pointer
    }
    return await client.store_pointer(dataset)
```

### Retrieving Datasets

```python
async def load_dataset(dataset_pointer: str) -> pd.DataFrame:
    client = await Client.init()
    
    # Retrieve dataset pointers
    dataset = await client.retrieve_pointer(dataset_pointer)
    
    # Load data and metadata
    data = await client.retrieve_chunk(dataset['data'])
    metadata = await client.retrieve_scratch_pad(dataset['metadata'])
    
    # Convert bytes to DataFrame
    return pd.read_parquet(data)
```

## Data Processing

### Parallel Processing

```python
from concurrent.futures import ThreadPoolExecutor
import numpy as np

async def process_chunks(
    data_pointer: str,
    chunk_size: int = 1000
) -> np.ndarray:
    client = await Client.init()
    data = await client.retrieve_chunk(data_pointer)
    df = pd.read_parquet(data)
    
    def process_chunk(chunk):
        # Your processing logic here
        return chunk.values
    
    chunks = [df[i:i+chunk_size] for i in range(0, len(df), chunk_size)]
    
    with ThreadPoolExecutor() as executor:
        results = list(executor.map(process_chunk, chunks))
    
    return np.concatenate(results)
```

### Feature Engineering

```python
from sklearn.preprocessing import StandardScaler
from typing import Dict, Any

async def engineer_features(
    dataset_pointer: str,
    feature_config: Dict[str, Any]
) -> str:
    client = await Client.init()
    df = await load_dataset(dataset_pointer)
    
    # Apply transformations
    for feature, config in feature_config.items():
        if config['type'] == 'standardize':
            scaler = StandardScaler()
            df[feature] = scaler.fit_transform(df[[feature]])
        elif config['type'] == 'log':
            df[feature] = np.log1p(df[feature])
    
    # Store transformed dataset
    return await store_dataset(
        df,
        name=f"transformed_{dataset_pointer}",
        description="Feature engineered dataset"
    )
```

## Model Training

### Training Pipeline

```python
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score
import pickle

async def train_model(
    dataset_pointer: str,
    model,
    test_size: float = 0.2
) -> str:
    # Load dataset
    df = await load_dataset(dataset_pointer)
    
    # Split features and target
    X = df.drop('target', axis=1)
    y = df['target']
    
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=test_size
    )
    
    # Train model
    model.fit(X_train, y_train)
    
    # Evaluate
    y_pred = model.predict(X_test)
    accuracy = accuracy_score(y_test, y_pred)
    
    # Store model
    model_bytes = pickle.dumps(model)
    model_pointer = await client.store_chunk(model_bytes)
    
    # Store metadata
    metadata = {
        'accuracy': accuracy,
        'test_size': test_size,
        'feature_names': list(X.columns),
        'model_type': str(type(model).__name__)
    }
    metadata_pointer = await client.store_scratch_pad(metadata)
    
    # Link model and metadata
    model_info = {
        'model': model_pointer,
        'metadata': metadata_pointer
    }
    return await client.store_pointer(model_info)
```

### Model Inference

```python
async def predict(
    model_pointer: str,
    data: pd.DataFrame
) -> np.ndarray:
    client = await Client.init()
    
    # Load model info
    model_info = await client.retrieve_pointer(model_pointer)
    
    # Load model and metadata
    model_bytes = await client.retrieve_chunk(model_info['model'])
    metadata = await client.retrieve_scratch_pad(model_info['metadata'])
    
    # Deserialize model
    model = pickle.loads(model_bytes)
    
    # Validate features
    expected_features = metadata['feature_names']
    if not all(f in data.columns for f in expected_features):
        raise ValueError("Missing required features")
    
    # Make predictions
    return model.predict(data[expected_features])
```

## Data Visualization

### Creating Visualizations

```python
import matplotlib.pyplot as plt
import seaborn as sns
from io import BytesIO

async def create_visualization(
    dataset_pointer: str,
    plot_type: str,
    **kwargs
) -> str:
    client = await Client.init()
    df = await load_dataset(dataset_pointer)
    
    # Create figure
    plt.figure(figsize=(10, 6))
    
    if plot_type == 'histogram':
        sns.histplot(data=df, **kwargs)
    elif plot_type == 'scatter':
        sns.scatterplot(data=df, **kwargs)
    elif plot_type == 'box':
        sns.boxplot(data=df, **kwargs)
    
    # Save plot to bytes
    buf = BytesIO()
    plt.savefig(buf, format='png')
    buf.seek(0)
    
    # Store visualization
    return await client.store_chunk(buf.read())
```

## Security and Privacy

### Data Encryption

```python
from cryptography.fernet import Fernet
from typing import Tuple

async def store_encrypted_dataset(
    df: pd.DataFrame,
    encryption_key: bytes
) -> Tuple[str, bytes]:
    # Generate encryption key if not provided
    if encryption_key is None:
        encryption_key = Fernet.generate_key()
    
    fernet = Fernet(encryption_key)
    
    # Convert DataFrame to bytes and encrypt
    data = df.to_parquet()
    encrypted_data = fernet.encrypt(data)
    
    # Store encrypted data
    client = await Client.init()
    pointer = await client.store_chunk(encrypted_data)
    
    return pointer, encryption_key
```

### Secure Collaboration

```python
async def share_dataset_access(
    dataset_pointer: str,
    collaborator_key: str
) -> str:
    client = await Client.init()
    
    # Create access control entry
    access = {
        'dataset': dataset_pointer,
        'granted_to': collaborator_key,
        'permissions': ['read', 'process']
    }
    
    # Store access control
    return await client.store_scratch_pad(access)
```

## Best Practices

1. Data Management
   - Use appropriate data formats (parquet for tabular data)
   - Implement proper versioning
   - Store metadata with datasets

2. Performance
   - Use chunked processing for large datasets
   - Implement caching for frequently accessed data
   - Optimize data formats and compression

3. Security
   - Encrypt sensitive data
   - Implement access controls
   - Monitor data access patterns

4. Collaboration
   - Document datasets and transformations
   - Share access securely
   - Track data lineage

## Resources

- [API Reference](../api/python/README.md)
- [Error Handling Guide](error_handling.md)
- [Quantum Security Guide](quantum_security.md)

## Support

For data science support:

- Technical support: [support@autonomi.com](mailto:support@autonomi.com)
- Documentation: [https://docs.autonomi.com/data-science](https://docs.autonomi.com/data-science)
- Community forum: [https://forum.autonomi.com/data-science](https://forum.autonomi.com/data-science)
