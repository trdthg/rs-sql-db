#include <stdio.h>

void CreateMyFile(char *szFileName, int nFileLength)
{
    FILE *fp = fopen(szFileName, "wb+"); // 创建文件
    if (fp == NULL)
        printf("文件打开失败");
    else
    {
        fseek(fp, nFileLength - 1, SEEK_SET); // 将文件的指针 移至 指定大小的位置
        fputc(0, fp);                        // 在要指定大小文件的末尾随便放一个数据
        fclose(fp);
    }
}

void main(int argc, char *argv[])

{

    CreateMyFile("test.txt",1024*256); //调用测试

    // FILE *fp = fopen("d.txt", "ab+");

    // int a = 0, b, i;

    // while (getc(fp) != EOF)

    //     a++;

    // for (i = 1; i < 1024 * 1024 * 1024 - a; i++)

    //     putc(0, fp);

    // fclose(fp);
}
