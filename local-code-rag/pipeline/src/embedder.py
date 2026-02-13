import ollama

EMBEDDING_MODEL = "nomic-embed-text"
BATCH_SIZE = 32


def embed_text(text: str, host: str = "http://localhost:11434") -> list[float]:
    """Embed a single text string using Ollama's nomic-embed-text model."""
    client = ollama.Client(host=host)
    response = client.embed(model=EMBEDDING_MODEL, input=text)
    return response["embeddings"][0]


def embed_batch(
    texts: list[str], host: str = "http://localhost:11434"
) -> list[list[float]]:
    """Embed multiple texts in a single call. Ollama supports batch input."""
    client = ollama.Client(host=host)
    response = client.embed(model=EMBEDDING_MODEL, input=texts)
    return response["embeddings"]


def embed_all(
    texts: list[str], host: str = "http://localhost:11434"
) -> list[list[float]]:
    """Embed a list of texts in batches, returns all embeddings in order."""
    all_embeddings = []
    for i in range(0, len(texts), BATCH_SIZE):
        batch = texts[i : i + BATCH_SIZE]
        embeddings = embed_batch(batch, host=host)
        all_embeddings.extend(embeddings)
        print(f"  Embedded {min(i + BATCH_SIZE, len(texts))}/{len(texts)} chunks")
    return all_embeddings
