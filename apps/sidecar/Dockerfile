FROM python:3.12-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY . .

# Default port
EXPOSE 8765

# Environment variables (can be overridden)
ENV HOST=0.0.0.0
ENV PORT=8765
ENV OLLAMA_HOST=http://ollama:11434

# Run the server
CMD ["python", "server.py"]
