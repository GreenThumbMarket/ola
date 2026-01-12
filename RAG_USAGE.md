# Ola RAG (Retrieval-Augmented Generation) Feature

The RAG feature allows you to index PDF and text documents and query them using natural language.

## Installation

First, build the project:
```bash
cargo build --release
```

## Usage

### 1. Index Documents

**Index a PDF file:**
```bash
ola rag index-pdf --file /path/to/document.pdf
```

**Index a text file:**
```bash
ola rag index-text --file /path/to/document.txt
```

### 2. Query Documents

**Retrieve relevant context:**
```bash
ola rag query --query "What is the main topic of the document?"
```

**Generate a full answer using the configured LLM:**
```bash
ola rag query --query "Explain the key concepts" --generate
```

### 3. Manage Index

**List all indexed documents:**
```bash
ola rag list
```

**View statistics:**
```bash
ola rag stats
```

**Clear all indexed documents:**
```bash
ola rag clear
# or force clear without confirmation
ola rag clear --force
```

## Configuration

Before using the RAG feature, make sure you have configured an LLM provider:

```bash
# Configure OpenAI (recommended for embeddings)
ola configure --provider OpenAI --api-key YOUR_API_KEY --model gpt-4

# Or configure another provider
ola configure
```

## How It Works

1. **Document Loading**: PDFs and text files are loaded and split into chunks
2. **Embedding Generation**: Each chunk is converted to a vector embedding using OpenAI's embedding API
3. **Vector Storage**: Embeddings are stored in a local vector database
4. **Retrieval**: When you query, your question is embedded and similar documents are found
5. **Generation**: The retrieved context is used to generate an accurate answer

## Examples

### Index and query a technical manual:
```bash
# Index the manual
ola rag index-pdf --file ~/Documents/user_manual.pdf

# Ask questions
ola rag query --query "How do I reset the device?" --generate
ola rag query --query "What are the safety precautions?" --generate
```

### Build a knowledge base:
```bash
# Index multiple documents
ola rag index-pdf --file doc1.pdf
ola rag index-pdf --file doc2.pdf
ola rag index-text --file notes.txt

# Check what's indexed
ola rag stats

# Query across all documents
ola rag query --query "Compare the approaches mentioned in the documents" --generate
```

## Notes

- The index is stored locally in `~/.ola/rag/`
- Embeddings use OpenAI's API by default (requires API key)
- The system retrieves the top 3 most relevant chunks by default
- Large PDFs may take some time to process