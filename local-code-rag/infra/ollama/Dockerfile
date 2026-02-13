FROM ollama/ollama:latest

RUN ollama serve & sleep 5 && ollama pull nomic-embed-text && pkill ollama

EXPOSE 11434
CMD ["serve"]
