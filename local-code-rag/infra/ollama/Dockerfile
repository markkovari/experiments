FROM ollama/ollama:latest

RUN ollama serve & sleep 5 && ollama pull nomic-embed-text && ollama pull qwen2.5-coder:7b && pkill ollama

EXPOSE 11434
CMD ["serve"]
