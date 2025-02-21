# nodisp
ビジュアルにノードを組んで作る自作Lisp系言語
- コンパイラ言語
- 自作バイトコードに変換
- 自作VMで実行
- 型推論

## 例1 単純な処理
<img width="1280" alt="Image" src="https://github.com/user-attachments/assets/217405bb-584c-40cd-b28f-b91183a337f8" />
<img width="1221" alt="Image" src="https://github.com/user-attachments/assets/0bc49e40-0417-4404-84e5-0c8e61195a60" />

### Python例

```python
a = 3
b = a + 5
print(a * b)
```

## 例2 階乗
<img width="1280" alt="スクリーンショット 2025-02-20 20 19 06" src="https://github.com/user-attachments/assets/820ba5c6-0ee4-49fc-a168-f4a01e3f3c16" />
<img width="1227" alt="スクリーンショット 2025-02-20 20 20 33" src="https://github.com/user-attachments/assets/fb5ebba2-ea33-46b2-a8cf-6c673b2f0738" />

### Python例

```python
def f(int n):
  res = -1
  if(n == 0) res = 1
  else res = n * f(n-1)
  return res
print(f(3))
```
