# must
Task runner in the spirt of **m** ake.  BTW written in r **ust** .

# Goal

```
# typical make style
target: dep
  shell line
  shell line
  
# multiple arg selection and pattern usage
target/{arg0}/{arg1}: file/of/${arg0}
  action >> ${arg1}
  
# plugins that support the is-change contract
docker/{image}: src/*
  ${DOCKER} build -t ${image} . 
```
### stretch
- [ ] simple make file compatability

## cli
list **real** targets
```
must -t
```

use specific file as grouped argument
```
must docker/run  # looks for docker.must and runs "run" task

must run         # looks for mustfile and runs "run" task
```

