# BroadlinkRemote API Dockerfile
# Purpose: Build a minimal image to run the Flask API + static UI

# 1) Base image
FROM python:3.14-slim AS runtime

# 2) Environment settings for reliable containerized Python
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    FLASK_HOST=0.0.0.0 \
    FLASK_PORT=5000

# 3) System deps (none strictly required for these Python libs, but keep pip up-to-date)
RUN python -m pip install --upgrade pip

# 4) Workdir
WORKDIR /app

# 5) Copy only the project files
# If you later add a requirements.txt, adjust the build to leverage Docker layer caching
COPY . /app

# 6) Python dependencies
# Minimal set per README; add others if your project grows
RUN pip install flask python-dotenv broadlink gunicorn

# 7) Expose the HTTP port
EXPOSE 5000

# 8) Default command
# Use Gunicorn in production; it will create the Flask app via the factory
# You can adjust workers/threads based on your environment
CMD ["gunicorn", "--factory", "-w", "2", "-b", "0.0.0.0:5000", "app_factory:create_app"]
