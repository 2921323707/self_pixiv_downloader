@echo off
setlocal
set NEXT_OUTPUT_EXPORT=1
npm.cmd --prefix ..\src\frontend run build
endlocal
