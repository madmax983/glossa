for file in $(find tests -name "*.rs"); do
  grep "use glossa::tools::" $file
done
