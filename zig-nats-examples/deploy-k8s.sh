#!/bin/bash
set -e

# Configuration
REPO="event-platform"
TAG=$(date +%s)

echo "🚀 Building microservices with tag: $TAG"
SERVICES=("auth" "org" "scheduler" "mock-executor" "gateway")

for SERVICE in "${SERVICES[@]}"; do
  # Map internal folder name to deployment image name if different
  IMG_NAME=$SERVICE
  if [ "$SERVICE" == "mock-executor" ]; then IMG_NAME="executor"; fi
  
  echo "📦 Building image for $SERVICE as $IMG_NAME..."
  docker build -f Dockerfile.service --build-arg SERVICE_NAME=$SERVICE -t $REPO/$IMG_NAME:$TAG .
done

echo "📦 Building image for frontend..."
docker build -f Dockerfile.frontend -t $REPO/frontend:$TAG .

echo "✅ All images built locally."

# Check if helm is installed
if ! command -v helm &> /dev/null
then
    echo "❌ Error: 'helm' is not installed. Please install it using 'brew install helm'."
    exit 1
fi

echo "☸️  Deploying to Kubernetes via Helm..."
helm upgrade --install event-platform ./charts/event-platform \
  --set services_base_repo=$REPO \
  --set image_tag=$TAG \
  --set frontend.image.tag=$TAG

echo "🎉 Deployment complete!"
echo "Check pods status with: kubectl get pods"
