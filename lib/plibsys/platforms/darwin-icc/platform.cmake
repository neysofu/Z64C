set (PLIBSYS_THREAD_MODEL posix)
set (PLIBSYS_IPC_MODEL posix)
set (PLIBSYS_TIME_PROFILER_MODEL posix)
set (PLIBSYS_DIR_MODEL posix)
set (PLIBSYS_LIBRARYLOADER_MODEL posix)

set (PLIBSYS_PLATFORM_LINK_LIBRARIES -pthread imf svml irng intlc)

set (PLIBSYS_PLATFORM_DEFINES
        -D_REENTRANT
)
