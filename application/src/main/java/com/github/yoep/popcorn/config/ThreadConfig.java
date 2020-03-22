package com.github.yoep.popcorn.config;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;
import org.springframework.scheduling.annotation.EnableAsync;
import org.springframework.scheduling.concurrent.ThreadPoolTaskExecutor;

@Configuration
@EnableAsync
public class ThreadConfig {
    private static final int CORE_POOL_SIZE = 2;
    private static final int QUEUE_SIZE = 0;
    private static final int THREAD_KEEP_ALIVE_SECONDS = 20;

    @Bean
    public TaskExecutor taskExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        executor.setCorePoolSize(CORE_POOL_SIZE);
        executor.setQueueCapacity(QUEUE_SIZE);
        executor.setKeepAliveSeconds(THREAD_KEEP_ALIVE_SECONDS);
        executor.setThreadNamePrefix("PT-background");
        executor.setWaitForTasksToCompleteOnShutdown(false);
        executor.initialize();
        return executor;
    }
}
