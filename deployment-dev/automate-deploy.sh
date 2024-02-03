kubectl create namespace load-balancer

echo ">>>>>> Creating shared persisting volume <<<<<<"
kubectl apply -f ./pers.yaml
sleep 1
echo ""
echo ">>>>>> Deploying cadvisor <<<<<<"
kubectl apply -f ./agent/cadvisor.yaml
sleep 1

echo ""
echo ">>>>>> Creating serviceaccount for the service discovery component <<<<<<"
kubectl apply -f ./service-discovery/customsd.yaml
sleep 1

echo ""
echo ">>>>>> Deploying the load balancer <<<<<<"
kubectl apply -f ./caching-db/redis-dep.yaml
kubectl apply -f ./service-discovery/sd-dep.yaml
kubectl apply -f ./reverse-proxy/rp-dep.yaml
kubectl apply -f ./api/api-dep.yaml
kubectl apply -f ./agent/agent-dep.yaml
