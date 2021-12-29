package com.github.yoep.popcorn.backend.utils;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class FileServiceTest {
    private FileService fileService;

    @BeforeEach
    void setUp() {
        fileService = new FileService();
    }

    @Test
    void testGetFile_whenPathIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> fileService.getFile(null), "path cannot be null");
    }

    @Test
    void testGetFile_whenPathIsEmpty_shouldReturnCurrentWorkingDirectory() {
        var expectedResult = System.getProperty("user.dir");

        var result = fileService.getFile("");

        assertEquals(expectedResult, result.getAbsolutePath());
    }

    @Test
    void testGetAbsolutePath_whenPathIsEmpty_shouldReturnTheWorkingDirectoryAbsolutePath() {
        var expectedResult = System.getProperty("user.dir");

        var result = fileService.getAbsolutePath("");

        assertEquals(expectedResult, result);
    }
}
