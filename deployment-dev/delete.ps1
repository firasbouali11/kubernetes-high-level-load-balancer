kubectl.exe delete -f .\reverse-proxy\rp-dep.yaml
kubectl.exe delete -f .\caching-db\redis-dep.yaml
kubectl.exe delete -f .\service-discovery\sd-dep.yaml
kubectl.exe delete -f .\agent\agent-dep.yaml
kubectl.exe delete -f .\api\api-dep.yaml
kubectl.exe delete -f .\pers.yaml
kubectl.exe delete -f .\service-discovery\customsd.yaml
kubectl.exe delete ns load-balancer
