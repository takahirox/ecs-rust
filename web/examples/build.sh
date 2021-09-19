for dir in $(ls -d ./*/)
do
  cd $dir
  bash build.sh
  cd -
done
