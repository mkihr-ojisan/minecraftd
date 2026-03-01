import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("base-conventions")
    id("com.google.protobuf") version "0.9.6"
}

java {
    sourceCompatibility = JavaVersion.VERSION_1_8
    targetCompatibility = JavaVersion.VERSION_1_8
}

tasks.withType<KotlinCompile>().configureEach {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_1_8)
    }
}

val protobufVersion = "4.33.5"

dependencies {
    implementation("com.kohlschutter.junixsocket:junixsocket-core:2.10.1")
    implementation("com.google.protobuf:protobuf-kotlin:$protobufVersion")
}

protobuf {
    generateProtoTasks {
        all().configureEach {
            builtins {
                create("kotlin")
            }
        }
    }
}
