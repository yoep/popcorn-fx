package com.github.yoep.torrent.stream.web;

import lombok.extern.slf4j.Slf4j;
import org.apache.catalina.connector.ClientAbortException;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.ControllerAdvice;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.bind.annotation.ResponseStatus;

@Slf4j
@ControllerAdvice
public class ControllerHandler {
    @ExceptionHandler
    @ResponseStatus(HttpStatus.INTERNAL_SERVER_ERROR)
    public void handle(Exception ex) {
        log.error(ex.getMessage(), ex);
    }

    @ExceptionHandler
    public void handle(ClientAbortException ex) {
        log.trace(ex.getMessage(), ex);
    }
}
