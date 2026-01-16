@rem
@rem
@rem Gradle启动脚本 - Windows
@rem

@if "%DEBUG%" == "" @set DEBUG=0

@rem 设置Java_HOME
@if not defined JAVA_HOME (
    set JAVA_HOME=C:\Program Files\Android\Android Studio\jbr
)

@rem 设置Android_HOME
@if not defined ANDROID_HOME (
    set ANDROID_HOME=C:\Users\Mechrevo\AppData\Local\Android\Sdk
)

@rem 启动Gradle
"%~dp0\gradle.bat" %*
