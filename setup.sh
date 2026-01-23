docker run -it --rm \
  -v $PWD:/work \
  -w /work \
  node:20 \
  npm install

if [ -d "/goinfre" ]; then
	mkdir -p /tmp/$USER/stdgames_target
	ln -s /tmp/$USER/stdgames_target src-tauri/target
else
	mkdir -p src-tauri/target
fi
