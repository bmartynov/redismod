# Redis task

## Caveats

* stop_timer on already stopped timer leads to redis crash
  ```
    redis-server(23003,0x10059c580) malloc: *** error for object 0x600001914120: pointer being freed was not allocated                                                                 
    redis-server(23003,0x10059c580) malloc: *** set a breakpoint in malloc_error_break to debug    
  ```
* dots in native data type name leads to
  ```Error: created data type is null```