@echo off
title PureRemove v1.2.1 - Build Release x64
cd /d "%~dp0"

echo.
echo  ====================================================
echo   PureRemove v1.2.1 - Compilation Release x64
echo  ====================================================
echo.

:: Cree le dossier logs si absent
if not exist ".logs" mkdir ".logs"

:: Verifie les prerequis
where node >nul 2>&1
if errorlevel 1 ( echo [ERREUR] Node.js non trouve. & pause & exit /b 1 )

where cargo >nul 2>&1
if errorlevel 1 ( echo [ERREUR] Cargo non trouve. Installez depuis rustup.rs & pause & exit /b 1 )

:: Verifie que le modele est present
if not exist "src-tauri\resources\model.onnx" (
    echo [ERREUR] model.onnx introuvable dans src-tauri\resources\
    echo          Telechargez RMBG-1.4 : huggingface.co/briaai/RMBG-1.4
    pause & exit /b 1
)

:: Installe les deps npm si absentes
if not exist "node_modules" (
    echo [INFO] node_modules absent - installation en cours...
    call npm install
    if errorlevel 1 ( echo [ERREUR] npm install echoue & pause & exit /b 1 )
    echo.
)

:: PRE-OPS : kill le process si en cours d'execution (evite "file in use")
tasklist /FI "IMAGENAME eq pure-remove.exe" 2>nul | find /I "pure-remove.exe" >nul
if not errorlevel 1 (
    echo [INFO] pure-remove.exe en cours - arret force...
    taskkill /F /IM "pure-remove.exe" >nul 2>&1
    timeout /t 1 /nobreak >nul
)

:: Ajoute la target x64 si necessaire
echo [1/3] Verification target x86_64-pc-windows-msvc...
rustup target add x86_64-pc-windows-msvc >nul 2>&1

:: BUILD : sortie affichee en direct ET sauvegardee dans le log
echo [2/3] Compilation release x64 (peut prendre 2-5 min)...
echo.

:: On passe par un script PS pour tee sans casser ERRORLEVEL
powershell -NoProfile -Command ^
  "$p = Start-Process -FilePath 'npx' -ArgumentList 'tauri','build','--target','x86_64-pc-windows-msvc' -NoNewWindow -Wait -PassThru; exit $p.ExitCode"
set BUILD_CODE=%ERRORLEVEL%

:: Si on veut aussi le log : relance avec redirect (optionnel, decommentez)
:: npx tauri build --target x86_64-pc-windows-msvc > ".logs\build.log" 2>&1

if %BUILD_CODE% neq 0 (
    echo.
    echo [ERREUR] Build echoue ^(code %BUILD_CODE%^).
    echo          Consultez la sortie ci-dessus pour le detail.
    echo.
    pause
    exit /b %BUILD_CODE%
)

:: VERIFY : localise les artefacts
echo.
echo [3/3] Localisation des artefacts...

set EXE=src-tauri\target\x86_64-pc-windows-msvc\release\pure-remove.exe
set NSIS_DIR=src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis

if exist "%EXE%" (
    echo.
    echo  [OK] Executable portable :
    echo       %~dp0%EXE%
) else (
    echo  [WARN] Executable non trouve a l'emplacement attendu.
)

if exist "%NSIS_DIR%" (
    echo.
    echo  [OK] Installeur NSIS :
    for /r "%NSIS_DIR%" %%f in (*.exe) do echo       %%f
)

:: Log horodate
echo [%DATE% %TIME%] Build v1.2.1 OK (code %BUILD_CODE%) >> ".logs\build.log"

echo.
echo  ====================================================
echo   Build termine avec succes !
echo  ====================================================
echo.
pause
