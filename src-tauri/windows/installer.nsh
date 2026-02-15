!macro NSIS_HOOK_POSTINSTALL
  ; --- 1. ASR 模型：已改为按需下载，不再打包进安装程序 ---
  ; SetCompress off
  ; SetOutPath "$INSTDIR\resources\asr_model"
  ; File "..\..\..\..\resources\asr_model\model.int8.onnx"
  ; File "..\..\..\..\resources\asr_model\tokens.txt"
  
  ; --- 2. DLL 文件：直接释放到根目录 (.exe 同级) ---
  SetOutPath "$INSTDIR"
  File "..\..\..\..\cargs.dll"
  File "..\..\..\..\onnxruntime.dll"
  File "..\..\..\..\onnxruntime_providers_shared.dll"
  File "..\..\..\..\sherpa-onnx-c-api.dll"
  File "..\..\..\..\sherpa-onnx-cxx-api.dll"

  ; --- 3. 恢复后续文件的压缩 ---
  SetCompress auto
!macroend
